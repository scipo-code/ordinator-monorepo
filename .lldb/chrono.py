from datetime import datetime, timedelta

def DateTimeSummary(valobj, internal_dict):
    ymdf = valobj.GetChildMemberWithName('datetime').GetChildMemberWithName('date').GetChildMemberWithName('ymdf').GetValue()
    secs = valobj.GetChildMemberWithName('datetime').GetChildMemberWithName('time').GetChildMemberWithName('time').GetValue()
    year = int(ymdf) >> 13
    day_of_year = (int(ymdf) & 8191 >> 4;
    date = datetime(year -1, 12, 31) + timedelta(days=day_of_year) + timedelta(seconds=int(secs))
    return date.strftime('%Y-%m-%d %H:%M:%S')
